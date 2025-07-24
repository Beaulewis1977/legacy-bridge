import React from 'react';
import { motion } from 'framer-motion';
import { CheckCircle2 } from 'lucide-react';

interface SuccessAnimationProps {
  message?: string;
}

export const SuccessAnimation: React.FC<SuccessAnimationProps> = ({ 
  message = 'Conversion completed!' 
}) => {
  return (
    <motion.div
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      exit={{ scale: 0, opacity: 0 }}
      transition={{
        type: "spring",
        stiffness: 260,
        damping: 20
      }}
      className="flex flex-col items-center justify-center p-6"
    >
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{
          delay: 0.2,
          type: "spring",
          stiffness: 200,
          damping: 10
        }}
        className="relative"
      >
        <motion.div
          className="absolute inset-0 bg-green-500/20 rounded-full"
          initial={{ scale: 0.8 }}
          animate={{ scale: 1.5, opacity: 0 }}
          transition={{
            duration: 0.6,
            delay: 0.3,
            ease: "easeOut"
          }}
        />
        <CheckCircle2 className="w-16 h-16 text-green-500 relative z-10" />
      </motion.div>
      
      <motion.p
        className="mt-4 text-lg font-medium text-center"
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
      >
        {message}
      </motion.p>
    </motion.div>
  );
};